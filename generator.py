import argparse
import random


def generate_data(filename: str, total_trees: int, tree_depth: int, node_degree: int):
    with open(filename, 'w') as file:
        file.write('parent, child, ownership\n')

        queue = [(to_entity_str(root_id), 0) for root_id in range(total_trees)]
        id_counter = total_trees

        while len(queue) > 0:
            (parent, level) = queue.pop(0)
            if level >= tree_depth - 1:
                continue

            for child_id in range(id_counter, id_counter + node_degree):
                child = to_entity_str(child_id)
                ownership = random.randrange(0, 10_001) / 10_000  # [0, 1]
                file.write(f'{parent},{child},{ownership}\n')
                queue.append((child, level + 1))
            id_counter = id_counter + node_degree


def to_entity_str(entity_id: int):
    return f'C{entity_id}'


if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description='Generates CSV data representing a forest (i.e., a graph of multiple trees).'
                    'The total number of nodes is given by the formula N * (D^L - 1) / (D - 1),'
                    'for a forest with N trees with degree D and depth L.')
    parser.add_argument('--trees', default=1, type=int, nargs=1, help='The total number of trees to generate.')
    parser.add_argument('--depth', default=10, type=int, nargs=1, help='The depth of every tree in the forest.')
    parser.add_argument('--degree', default=3, type=int, nargs=1, help='The node degree of every node in every tree.')
    parser.add_argument('output', type=str, help='The file to which to write the CSV data.')
    args = parser.parse_args()

    random.seed(0)  # deterministic pseudo-random numbers
    trees, depth, degree = args.trees[0], args.depth[0], args.degree[0]
    generate_data(args.output, trees, depth, degree)
    if degree == 1:
        total_edges = trees * depth
    else:
        total_edges = trees * (pow(degree, depth) - 1) / (degree - 1)
    print(f'Generated {trees} tree(s) with depth {depth} and node degree {degree}, {total_edges} edges in total.')