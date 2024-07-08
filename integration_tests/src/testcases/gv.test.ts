import path from 'path';
import { describe, expect, test } from 'vitest';
import fs from 'fs';
import dotparser, { Graph } from 'dotparser';

type MyNode = {
  id: string;
  label: string;
  shape: string;
  outs: string[];
};

class ParsedGraph {
  nodes: Record<string, MyNode> = {};
  constructor(content: string) {
    this.nodes = parseAST(dotparser(content));
  }
  hasPath(checkNodes: string[]) {
    if (!checkNodes || !checkNodes.length) {
      return false;
    }
    const node = this.nodes[checkNodes.shift()];
    if (!node) {
      return false;
    }
    if (node.outs.length === 0 || !node.outs.includes(checkNodes[0])) {
      return false;
    }
    if (checkNodes.length === 1) {
      return true;
    }
    return this.hasPath(checkNodes);
  }
}

const parseAST = (tree: Graph[]): Record<string, MyNode> => {
  const out: Record<string, MyNode> = {};
  if (!tree || !tree.length) {
    throw new Error('No tree found');
  }
  const graph = tree[0];
  for (const item of graph.children) {
    if (item.type === 'subgraph') {
      for (const subItem of item.children) {
        if (subItem.type === 'node_stmt') {
          out[subItem.node_id.id] = {
            id: subItem.node_id.id.toString(),
            label: subItem.attr_list
              .find((attr) => attr.id === 'label')
              ?.eq.toString(),
            shape:
              subItem.attr_list
                .find((attr) => attr.id === 'shape')
                ?.eq.toString() || 'ellipse',
            outs: [],
          };
        }
      }
    }
  }
  for (const item of graph.children) {
    if (item.type === 'node_stmt') {
      out[item.node_id.id] = {
        id: item.node_id.id.toString(),
        label: item.attr_list
          .find((attr) => attr.id === 'label')
          ?.eq.toString(),
        shape:
          item.attr_list.find((attr) => attr.id === 'shape')?.eq.toString() ||
          'ellipse',
        outs: [],
      };
    }
  }

  for (const item of graph.children) {
    if (item.type === 'edge_stmt') {
      const firstId = item.edge_list[0].id.toString();
      const secondId = item.edge_list[1].id.toString();
      out[firstId].outs.push(secondId);
    }
  }
  return out;
};

describe('graphviz', () => {
  test('init', () => {
    const gvFile = path.resolve(__dirname, '../../../graph.gv');
    const gvContent = fs.readFileSync(gvFile, 'utf-8');
    const parsedTree = new ParsedGraph(gvContent);
    expect(parsedTree).toBeDefined();
    expect(parsedTree.hasPath(['a', 'b'])).toBe(false);
    expect(
      parsedTree.hasPath([
        'K000',
        'K002',
        'K003',
        'K005',
        'K007',
        'K045',
        'K009',
        'K010',
        'K011',
        'K012',
        'K004',
      ]),
    ).toBe(false);
    expect(
      parsedTree.hasPath([
        'K000',
        'K002',
        'K003',
        'K004',
        'K005',
        'K007',
        'K048',
        'K009',
        'K010',
        'K011',
        'K012',
        'K047',
        'K013',
        'K014',
        'K015',
        'K016',
        'K017',
      ]),
    ).toBe(true);
  });
});
