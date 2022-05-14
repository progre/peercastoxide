import { IGroup } from '@fluentui/react';
import { Tree } from '../TreesView';

function parse<T>(
  atom: Tree<T>,
  groupNameKey: string,
  level: number,
  startIndex: number
): { items: T[]; group: IGroup } {
  let items: T[];
  let children: IGroup[] | undefined;
  if ('children' in atom) {
    items = [];
    children = [];
    let childrenStartIndex = startIndex;
    atom.children.forEach((x) => {
      const { items: itemItems, group: itemGroup } = parse(
        x,
        groupNameKey,
        level + 1,
        childrenStartIndex++
      );
      items = [...items, ...itemItems];
      children = [...children!!, itemGroup];
    });
  } else {
    items = [atom];
  }
  const group: IGroup = {
    key: `${level}_${startIndex}`,
    name: (atom as any)[groupNameKey],
    startIndex: startIndex,
    count: items.length,
    isCollapsed: children != null,
    level,
    children,
  };
  return { items, group };
}

export default function parseTree<T>(
  trees: readonly Tree<T>[],
  groupNameKey: string
): {
  items: T[];
  groups: IGroup[];
} {
  const items: T[] = [];
  const groups: IGroup[] = [];

  trees.forEach((atom, i) => {
    const { items: itemItems, group } = parse(
      atom,
      groupNameKey,
      0,
      items.length
    );
    items.push(...itemItems);
    groups.push(group);
  });
  return { items, groups };
}
