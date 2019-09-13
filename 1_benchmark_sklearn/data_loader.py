import multiprocessing as mp

def convert_line(line):
    (x, y) = line.strip().split(",")
    x      = float(x)
    y      = float(y)

    return (x, y)

def load(filepath):
    with open(filepath, "r") as f:
        content = f.readlines()

    with mp.Pool() as p:
        all_points = p.map(convert_line, content)

    return all_points


def batched_ranges(allData, numProcesses):
    step       = 1
    batchSize  = round(len(allData) / numProcesses)
    firstRange = range(0, batchSize, step)
    ranges     = [firstRange]

    if (numProcesses > 2):
        midRanges = [range((i-1)*batchSize, batchSize*i, step) for i in range(2, numProcesses)]
        ranges.extend(midRanges)

    finalRange= range((numProcesses-1)*batchSize, len(allData), step)
    ranges.append(finalRange)

    return ranges