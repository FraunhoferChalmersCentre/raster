from collections import Counter
from enum import Enum, auto
import multiprocessing as mp



class PARALLELIZE(Enum):
    MP_PROCESS    = auto()
    MP_POOL       = auto()
    POOL_EXECUTOR = auto()


def worker(inpt, output):
    for fun, args in iter(inpt.get, 'STOP'):
        result = fun(*args)
        output.put(result)


# This is the computation each worker will perform
def partial_projection(partial_range, scalar):
    my_tiles = dict()
    for i in partial_range:
        lat, lon             = shared_array[i] # read globally shared array
        coordinate           = (int(lat*scalar), int(lon*scalar))

        numPointsInTile      = my_tiles.get(coordinate, 0)
        my_tiles[coordinate] = numPointsInTile + 1

    return my_tiles


# The prime version retains all points in each tile
def partial_projection_prime(partial_range, scalar):
    my_tiles = dict()
    for i in partial_range:
        lat, lon             = shared_array[i] # read globally shared array
        coordinate           = (int(lat*scalar), int(lon*scalar))

        pointsInTile         = my_tiles.get(coordinate, [])
        pointsInTile.append((lat, lon))
        my_tiles[coordinate] = pointsInTile

    return my_tiles



# Variant PARALLELIZE.POOL_EXECUTOR
def processPoolExecutor(points, batches, scalar):
    import concurrent.futures as futures
    global shared_array
    shared_array = points

    all_points = Counter()
    with futures.ProcessPoolExecutor(max_workers=len(batches)) as executor:
        fs = []
        for a_range in batches:
            fs.append(executor.submit(partial_projection, a_range, scalar))
        for f in fs:
            all_points.update(f.result())

    return all_points


# Variant PARALLELIZE.MP_POOL
def pool(points, batches, scalar):
    global shared_array
    shared_array = points

    all_points = Counter()
    with mp.Pool(len(batches)) as p:
        fs = []
        for a_range in batches:
            fs.append(p.apply_async(partial_projection, (a_range, scalar)))
        for f in fs:
            all_points.update(f.get())

    return all_points


# Variant PARALLELIZE.MP_PROCESS
def process(points, batches, scalar):
    global shared_array
    shared_array = points

    task_queue   = mp.Queue()
    result_queue = mp.Queue()
    worker_iter  = range(len(batches))

    # queue all tasks, one task per batch
    for a_range in batches:
        task_queue.put((partial_projection, (a_range, scalar)))

    # start workers that sequentially process tasks from task_queue
    for _ in worker_iter:
        mp.Process(target=worker, args=(task_queue, result_queue)).start()

    # collect and merge results from each process
    all_points  = result_queue.get()
    for _ in range(len(batches)-1):
        partial_count = result_queue.get()
        for k, v in partial_count.items():
            num = all_points.get(k, 0)
            all_points[k] = num + v

    # tell workers to stop
    for _ in worker_iter:
        task_queue.put('STOP')

    return all_points


# For use with raster_prime, i.e retain all points in each tile
# rather than just counting them.
def process_prime(points, batches, scalar):
    global shared_array
    shared_array = points

    task_queue   = mp.Queue()
    result_queue = mp.Queue()
    worker_iter  = range(len(batches))

    # queue all tasks, one task per batch
    for a_range in batches:
        task_queue.put((partial_projection_prime, (a_range, scalar)))

    # start workers that sequentially process tasks from task_queue
    for _ in worker_iter:
        mp.Process(target=worker, args=(task_queue, result_queue)).start()

    # collect and merge results from each process
    all_points  = result_queue.get()
    for _ in range(len(batches)-1):
        partial_dict = result_queue.get()
        for k, v in partial_dict.items():
            points = all_points.get(k, [])
            points.extend(v)
            all_points[k] = points

    # tell workers to stop
    for _ in worker_iter:
        task_queue.put('STOP')

    return all_points
