import h5py
import argparse
import flatnav
import tqdm


def main():
    parser = argparse.ArgumentParser(description="Convert dataset to HDF5 format.")
    parser.add_argument("-d", "--dataset", type=str)
    args = parser.parse_args()

    data = {}

    with h5py.File(args.dataset, "r") as f:
        for key in f.keys():
            data[key] = f[key][()]
            print(f"loaded {key} with shape {data[key].shape}")

    train = data["train"]
    test = data["test"]
    groundtruths = data["neighbors"]

    index = flatnav.IndexEuclideanF32(32, train.shape[1], train.shape[0])

    for i, sample in tqdm.tqdm(enumerate(train), total=len(train)):
        index.insert(i, sample, 64)

    index.reorder(flatnav.GOrder(w=10))

    tp = 0
    fp = 0
    fn = 0

    for query, gtruths in tqdm.tqdm(zip(test, groundtruths), total=len(test)):
        results = index.query(query, 64, 10)

        results = set(x[0] for x in results)
        gtruths = set(gtruths[:10])

        tp += len(results.intersection(gtruths))
        fp += len(results.difference(gtruths))
        fn += len(gtruths.difference(results))

    print(f"precision@10 {tp / (tp + fp)}")
    print(f"recall@10 {tp / (tp + fn)}")


if __name__ == "__main__":
    main()
