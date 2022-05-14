import { css } from '@emotion/react';
import {
  CollapseAllVisibility,
  DetailsRow,
  GroupedList,
  IColumn,
  IDetailsColumnFieldProps,
  IGroup,
  IGroupHeaderProps,
  IRenderFunction,
  SelectionMode,
} from '@fluentui/react';
import parseTree from './trees-view/parseTree';

const GroupHeader: IRenderFunction<IGroupHeaderProps> = (
  props,
  defaultRender
) =>
  props?.group?.children == null ||
  props?.group?.children?.length === 0 ? null : (
    <div
      css={css`
        /* グループヘッダーの高さを要素に合わせる */
        > div > div,
        > div > div > div > button {
          height: 32px;
        }

        /* 非ボタンとボタンのカーソル適正化 */
        > div > div > div > button {
          cursor: pointer;
        }
        > div > div * {
          cursor: initial;
        }

        /* ホバー廃止 */
        > div:hover {
          background-color: initial;
        }

        > div > div > div:nth-of-type(3) > span {
          font-family: monospace;
        }
      `}
    >
      {defaultRender?.(props) ?? null}
    </div>
  );

export type Tree<T> = T | { children: Tree<T>[] };

export default function TreesView<T>(props: {
  trees: readonly T[];
  identifierKey: string;
  payloadKey: string;
  onRenderField?: IRenderFunction<IDetailsColumnFieldProps>;
}): JSX.Element {
  const { items, groups } = parseTree(props.trees, props.identifierKey);

  return (
    <GroupedList
      groups={groups}
      items={items}
      selectionMode={SelectionMode.none}
      compact={true}
      groupProps={{ onRenderHeader: GroupHeader }}
      onRenderCell={(
        _nestingDepth?: number,
        item?: T,
        itemIndex?: number,
        group?: IGroup
      ): React.ReactNode =>
        item == null ||
        typeof itemIndex !== 'number' ||
        itemIndex < 0 ? null : (
          <DetailsRow
            columns={[props.identifierKey, props.payloadKey].map(
              (fieldName): IColumn => ({
                key: fieldName,
                name: '__unused__',
                fieldName,
                minWidth: 100,
              })
            )}
            groupNestingDepth={(group?.level ?? 0) + 2}
            item={item}
            itemIndex={itemIndex}
            selectionMode={SelectionMode.none}
            compact={true}
            group={group}
            collapseAllVisibility={CollapseAllVisibility.hidden}
            onRenderField={props.onRenderField}
            css={css`
              /* ホバー廃止 */
              :hover {
                background-color: initial;
              }

              > div > div:first-of-type {
                font-family: monospace;
              }
            `}
          />
        )
      }
    />
  );
}
