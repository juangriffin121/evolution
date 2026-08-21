"""
Reads the .bin file produced by FrameWriter (src/mods/frames.rs).

File layout (repeated until EOF), written little-endian:
    age         : u64 (8 bytes)
    blob_count  : u32 (4 bytes)
    blob_count * {
        x       : f32 (4 bytes)
        y       : f32 (4 bytes)
        type    : u8  (1 byte)   0 = Prey, 1 = Predator
        energy  : f32 (4 bytes)
    }

Each frame can have a DIFFERENT blob_count (births/deaths change the
population), so this is a variable-length record stream -- you can't
np.fromfile() the whole thing with one dtype. Parse it frame by frame:
read the small header to find out how many blob records follow, then
let numpy read just that slice in one shot.
"""
import struct
import numpy as np

HEADER = struct.Struct('<QI')          # age (u64) + blob_count (u32) = 12 bytes
BLOB_DTYPE = np.dtype([
    ('x', '<f4'),
    ('y', '<f4'),
    ('type', 'u1'),                    # 0 = Prey, 1 = Predator
    ('energy', '<f4'),
])                                      # 13 bytes/blob, tightly packed (verified, no padding)


def read_frames(path):
    """Yield (age, structured_array_of_blobs) for every frame in the file."""
    with open(path, 'rb') as f:
        data = f.read()

    offset = 0
    n = len(data)
    while offset < n:
        age, count = HEADER.unpack_from(data, offset)
        offset += HEADER.size

        nbytes = count * BLOB_DTYPE.itemsize
        blobs = np.frombuffer(data, dtype=BLOB_DTYPE, count=count, offset=offset)
        offset += nbytes

        yield age, blobs


if __name__ == "__main__":
    import sys
    path = sys.argv[1] if len(sys.argv) > 1 else "./runs/frames/12.bin"
    for age, blobs in read_frames(path):
        prey = (blobs['type'] == 0).sum()
        pred = (blobs['type'] == 1).sum()
        print(f"age={age:5d}  n={len(blobs):4d}  prey={prey:4d}  pred={pred:4d}")
