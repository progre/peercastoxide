import { css } from '@emotion/react';
import { useEffect, useRef, useState } from 'react';
import {
  FixedSizeNodeComponentProps,
  VariableSizeNodeComponentProps,
  VariableSizeNodeData,
  VariableSizeTree,
} from 'react-vtree';
import { NodeData, TreeWalker } from 'react-vtree/dist/es/Tree';

interface TreeNodeData<T> extends NodeData {
  nestingLevel: number;
  payload: T;
}

type NestedData<T> = T & { children?: readonly NestedData<T>[] };

function createTreeWalker<T>(
  trees: readonly NestedData<T>[]
): TreeWalker<T & NodeData & VariableSizeNodeData & { nestingLevel: number }> {
  return function* treeWalker(
    refresh: boolean
  ): Generator<
    | (T & NodeData & VariableSizeNodeData & { nestingLevel: number })
    | string
    | symbol,
    void,
    boolean
  > {
    const stack = trees.map((tree, index) => ({
      ...tree,
      index,
      parentId: '',
      nestingLevel: 0,
    }));
    stack.reverse();

    while (stack.length !== 0) {
      const item = stack.pop()!;
      const { index, parentId, nestingLevel, children } = item;
      const id = `${parentId}-${index}`;

      const isOpened = yield !refresh
        ? id
        : {
            ...item,
            id,
            isOpenByDefault: false,
            defaultHeight: 30,
            nestingLevel,
          };
      if (children != null && children.length !== 0 && isOpened) {
        for (let i = children.length - 1; i >= 0; i--) {
          stack.push({
            ...children[i],
            index: i,
            parentId: id,
            nestingLevel: nestingLevel + 1,
          });
        }
      }
    }
  };
}

export default function TreesView<
  T extends { children?: readonly T[] }
>(props: {
  trees: readonly T[];
  onRenderField: (
    props: VariableSizeNodeComponentProps<
      T & NodeData & VariableSizeNodeData & { nestingLevel: number }
    >
  ) => JSX.Element;
}): JSX.Element {
  const treeWalker = createTreeWalker(props.trees);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const [height, setHeight] = useState(0);
  useEffect(() => {
    if (wrapperRef.current == null) {
      return;
    }
    const onResize = () => {
      setHeight(wrapperRef.current?.clientHeight ?? 0);
    };
    window.addEventListener('resize', onResize);
    setHeight(wrapperRef.current.clientHeight);
    return () => {
      window.removeEventListener('resize', onResize);
    };
  }, [wrapperRef.current]);
  return (
    <div
      ref={wrapperRef}
      css={css`
        height: 100%;
      `}
    >
      <VariableSizeTree
        treeWalker={treeWalker}
        // itemSize={30}
        height={height}
        width={'100%'}
      >
        {props.onRenderField}
      </VariableSizeTree>
    </div>
  );
}
