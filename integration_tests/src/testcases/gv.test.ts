import path from 'path';
import { describe, expect, test } from 'vitest';
import fs from 'fs';
import { ParsedGraph } from '../helpers/graphvis';

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
