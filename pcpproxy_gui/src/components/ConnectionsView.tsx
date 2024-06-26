import { css } from '@emotion/react';
import { DefaultButton, Dropdown, Icon, ResponsiveMode } from '@fluentui/react';
import { useCallback, useEffect, useState } from 'react';
import {
  VariableSizeNodeComponentProps,
  VariableSizeNodeData,
} from 'react-vtree';
import { NodeData } from 'react-vtree/dist/es/Tree';
import { Atom, AtomChild, AtomParent } from '../App';
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

function AtomParentView(props: {
  data: AtomParent;
  isOpen: boolean;
  toggle(): void;
}): JSX.Element {
  return (
    <DefaultButton
      onClick={() => {
        props.toggle();
      }}
      css={css`
        border: none;
        margin-top: 1px;
        height: 29px;

        > span > * {
          display: flex;
          align-items: center;
          height: 16px;
        }
      `}
    >
      <Icon
        iconName={props.isOpen ? 'chevrondown' : 'chevronrightmed'}
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
        <Identifier identifier={props.data.identifier} />
      </div>
    </DefaultButton>
  );
}

function AtomChildView(props: { data: AtomChild }): JSX.Element {
  return (
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
        <Identifier identifier={props.data.identifier} />
      </div>
      <div
        css={css`
          margin-left: 1em;
          white-space: pre;
          font-size: 13px;
          line-height: 16px;
        `}
      >
        {typeof props.data.payload === 'string'
          ? props.data.payload.replaceAll('\r', '␍').replaceAll('\n', '␊\n')
          : ['newp', 'pos\0', 'oldp'].includes(props.data.identifier) &&
            typeof props.data.payload === 'number'
          ? new Intl.NumberFormat().format(props.data.payload)
          : typeof props.data.payload === 'object'
          ? props.data.payload?.join?.(', ')
          : props.data.payload}
      </div>
    </div>
  );
}

function Raw(props: { identifier: '#RAW' | '#IFO'; payload: string }) {
  return (
    <div
      css={css`
        height: 32px;
        margin-left: 40px;
        display: flex;
        white-space: pre;
        ${props.identifier !== '#IFO' ? null : 'color: #ff9900;'}
        ${props.identifier !== '#IFO' ? null : 'font-style: italic;'}

        display: flex;
        align-items: center;
        ${props.payload.split('\n').length <= 0
          ? null
          : 'font-size: 13px; line-height: 16px;'}
      `}
    >
      {props.payload.replaceAll('\r', '␍').replaceAll('\n', '␊\n')}
    </div>
  );
}

function Field(
  props: VariableSizeNodeComponentProps<
    Atom & NodeData & VariableSizeNodeData & { nestingLevel: number }
  >
): JSX.Element {
  useEffect(() => {
    if ('children' in props.data || typeof props.data.payload !== 'string') {
      props.resize(32, true);
      return;
    }
    const lines = props.data.payload.split('\n').length;
    if (lines <= 1) {
      props.resize(32, true);
      return;
    }
    props.resize(lines * 16, true);
  }, [props.resize, props.data, props.height]);
  return (
    <div
      style={{
        ...props.style,
        alignItems: 'center',
        paddingLeft: `${props.data.nestingLevel * 32}px`,
        display: 'flex',
      }}
      css={css`
        border-bottom: 1px solid #ccc;
      `}
    >
      {'children' in props.data ? (
        <AtomParentView
          data={props.data as AtomParent}
          isOpen={props.isOpen}
          toggle={props.toggle}
        />
      ) : ['#RAW', '#IFO'].includes(props.data.identifier) ? (
        <Raw
          identifier={props.data.identifier as any}
          payload={props.data.payload as string}
        />
      ) : (
        <AtomChildView data={props.data} />
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
      <div
        css={css`
          margin: 0 8px;
        `}
      >
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
}): JSX.Element | null {
  const [selectedConnectionKey, setSelectedConnectionKey] = useState<
    string | null
  >(() => null);

  if (
    selectedConnectionKey != null &&
    props.connections[selectedConnectionKey!] == null
  ) {
    setSelectedConnectionKey(null);
    return null;
  }

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
        css={css`
          margin: 8px;
        `}
        options={Object.entries(props.connections).map(([key, value]) => ({
          key,
          text: `${value.clientHost} -> ${value.serverHost}`,
        }))}
        responsiveMode={ResponsiveMode.large}
        selectedKey={selectedConnectionKey}
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
        {selectedConnectionKey == null ||
        props.connections[selectedConnectionKey] == null ? null : (
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
