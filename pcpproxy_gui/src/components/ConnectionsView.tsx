import { css } from '@emotion/react';
import { Dropdown } from '@fluentui/react';
import { useState } from 'react';
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

function AtomStreamView(props: {
  label: string;
  atomStream: readonly Atom[];
}): JSX.Element {
  return (
    <div>
      {props.label}
      <TreesView
        trees={props.atomStream}
        identifierKey="identifier"
        payloadKey="payload"
        onRenderField={(props, defaultRender) => (
          <div
            key={props?.column.fieldName}
            css={css`
              color: rgb(32, 31, 30);
              font-size: 14px;

              > div {
                display: flex;
                align-items: center;
              }
            `}
          >
            {defaultRender?.(props) ?? null}
          </div>
        )}
      />
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
        display: flex;
      `}
    >
      <div
        css={css`
          flex-grow: 1;
          overflow-x: scroll;
        `}
      >
        <AtomStreamView
          label={`Client: ${props.clientHost} 上り`}
          atomStream={props.uploadStream}
        />
      </div>
      <div
        css={css`
          flex-grow: 1;
          overflow-x: scroll;
        `}
      >
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
  const [selectedConnection, setSelectedConnection] =
    useState<Connection | null>(null);
  return (
    <div>
      <Dropdown
        options={Object.entries(props.connections).map(([key, value]) => ({
          key,
          text: `${value.clientHost} -> ${value.serverHost}`,
        }))}
        onChange={(_ev, option) => {
          if (option == null) {
            return;
          }
          setSelectedConnection(props.connections[option.key as string]);
        }}
      />
      {selectedConnection == null ? null : (
        <ConnectionView
          clientHost={selectedConnection.clientHost}
          serverHost={selectedConnection.serverHost}
          uploadStream={selectedConnection.uploadStream}
          downloadStream={selectedConnection.downloadStream}
        />
      )}
    </div>
  );
}
