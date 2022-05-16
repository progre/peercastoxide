import { css } from '@emotion/react';
import { DefaultButton, Dropdown, Icon } from '@fluentui/react';
import { useState } from 'react';
import {
  VariableSizeNodeComponentProps,
  VariableSizeNodeData,
} from 'react-vtree';
import { NodeData } from 'react-vtree/dist/es/Tree';
import { Atom } from '../App';
import TreesView from './TreesView';

export interface Connection {
  clientHost: string;
  serverHost: string;
  uploadStream: readonly Atom[];
  downloadStream: readonly Atom[];
}

export type Connections = {
  [clientHostServerHost: string]: Connection;
};

function Identifier(props: { identifier: string }): JSX.Element {
  return (
    <span
      css={css`
        font-family: monospace;
      `}
    >
      {props.identifier}
    </span>
  );
}

function Field({
  data,
  isOpen,
  style,
  toggle,
}: VariableSizeNodeComponentProps<
  Atom & NodeData & VariableSizeNodeData & { nestingLevel: number }
>): JSX.Element {
  return (
    <div
      style={{
        ...style,
        alignItems: 'center',
        marginLeft: `${data.nestingLevel * 32}px`,
        display: 'flex',
      }}
    >
      {'children' in data ? (
        <DefaultButton
          onClick={toggle}
          css={css`
            border: none;

            > span > * {
              display: flex;
              align-items: center;
              height: 16px;
            }
          `}
        >
          <Icon
            iconName={isOpen ? 'chevrondown' : 'chevronrightmed'}
            css={css`
              width: 16px;
            `}
          />
          <div
            css={css`
              margin-left: 8px;
              width: 32px;
            `}
          >
            <Identifier identifier={data.identifier} />
          </div>
        </DefaultButton>
      ) : (
        <div
          css={css`
            height: 32px;
            margin-left: 40px;
            display: flex;

            > div {
              display: flex;
              align-items: center;
            }
          `}
        >
          <div>
            <Identifier identifier={data.identifier} />
          </div>
          <div
            css={css`
              margin-left: 1em;
              white-space: nowrap;
            `}
          >
            {data.payload}
          </div>
        </div>
      )}
    </div>
  );
}

function AtomStreamView(props: {
  label: string;
  atomStream: readonly Atom[];
}): JSX.Element {
  return (
    <div
      css={css`
        height: 100%;
        overflow: hidden;
        display: flex;
        flex-direction: column;
      `}
    >
      <div>
        {props.label} ({props.atomStream.length})
      </div>
      <div
        css={css`
          flex-grow: 1;
          overflow: hidden;
        `}
      >
        <TreesView trees={props.atomStream} onRenderField={Field} />
      </div>
    </div>
  );
}

function ConnectionView(props: {
  clientHost: string;
  serverHost: string;
  uploadStream: readonly Atom[];
  downloadStream: readonly Atom[];
}): JSX.Element {
  return (
    <div
      css={css`
        height: 100%;
        overflow: hidden;
        display: flex;
        overflow: hidden;

        > div {
          flex-grow: 1;
          height: 100%;
          overflow: hidden;
        }
      `}
    >
      <div>
        <AtomStreamView
          label={`Client: ${props.clientHost} 上り`}
          atomStream={props.uploadStream}
        />
      </div>
      <div>
        <AtomStreamView
          label={`Server: ${props.serverHost} 下り`}
          atomStream={props.downloadStream}
        />
      </div>
    </div>
  );
}

export default function ConnectionsView(props: {
  connections: Connections;
}): JSX.Element {
  const [selectedConnectionKey, setSelectedConnectionKey] = useState<
    string | null
  >(null);
  return (
    <div
      css={css`
        height: 100%;
        overflow: hidden;
        display: flex;
        flex-direction: column;
      `}
    >
      <Dropdown
        options={Object.entries(props.connections).map(([key, value]) => ({
          key,
          text: `${value.clientHost} -> ${value.serverHost}`,
        }))}
        onChange={(_ev, option) => {
          if (option == null) {
            return;
          }
          setSelectedConnectionKey(option.key as string);
        }}
      />
      <div
        css={css`
          flex-grow: 1;
          overflow: hidden;
        `}
      >
        {selectedConnectionKey == null ? null : (
          <ConnectionView
            clientHost={props.connections[selectedConnectionKey].clientHost}
            serverHost={props.connections[selectedConnectionKey].serverHost}
            uploadStream={props.connections[selectedConnectionKey].uploadStream}
            downloadStream={
              props.connections[selectedConnectionKey].downloadStream
            }
          />
        )}
      </div>
    </div>
  );
}
