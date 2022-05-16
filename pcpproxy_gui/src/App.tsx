import { css } from '@emotion/react';
import {
  Dropdown,
  GroupedList,
  SpinButton,
  Spinner,
  TextField,
} from '@fluentui/react';
import { invoke } from '@tauri-apps/api';
import { listen } from '@tauri-apps/api/event';
import { useEffect, useState } from 'react';
import ConnectionsView, { Connections } from './components/ConnectionsView';
import TreesView from './components/TreesView_';
import TreesView2 from './components/TreesView';
import dummyData from './utils/dummyData';

export type AtomOrRaw = Atom | string;

export type Atom = { children?: readonly Atom[] } & (AtomParent | AtomChild);

export interface AtomParent {
  identifier: string;
  children: readonly Atom[];
}

export interface AtomChild {
  identifier: string;
  payload: string;
}

function updatedConnections(
  connections: Connections,
  payload: JsonPayload
): Connections {
  let atom: Atom;
  if (payload.type === 'atom') {
    atom = payload.payload as Atom;
  } else if (payload.type === 'raw') {
    atom = {
      identifier: '#RAW',
      payload: payload.payload as string,
    };
  } else {
    atom = {
      identifier: '#IFO',
      payload: payload.type,
    };
  }
  const key = `${payload.clientHost}_${payload.serverHost}`;
  let connection = connections[key] ?? {
    clientHost: payload.clientHost,
    serverHost: payload.serverHost,
    uploadStream: [],
    downloadStream: [],
  };
  switch (payload.direction) {
    case 'upload':
      connection = {
        ...connection,
        uploadStream: [...connection.uploadStream, atom],
      };
      break;
    case 'download':
      connection = {
        ...connection,
        downloadStream: [...connection.downloadStream, atom],
      };
      break;
    default:
      throw new Error();
  }
  return { ...connections, [key]: connection };
}

export interface JsonPayload {
  type: string;
  clientHost: string;
  serverHost: string;
  direction: string;
  payload: AtomOrRaw | null;
}

export default function App(): JSX.Element {
  const [connections, setConnections] = useState<{
    [clientHostServerHost: string]: {
      clientHost: string;
      serverHost: string;
      uploadStream: readonly Atom[];
      downloadStream: readonly Atom[];
    };
  }>({});
  useEffect(() => {
    dummyData.forEach((x) => {
      setConnections((connections) => updatedConnections(connections, x));
    });

    const unlistenFnPromise = listen<JsonPayload>('json', (ev) => {
      setConnections((connections) =>
        updatedConnections(connections, ev.payload)
      );
    });
    (async () => {
      // const initialData = await invoke('initial_data');
    })();

    return () => {
      unlistenFnPromise.then((x) => x());
    };
  }, []);

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
        <SpinButton
          label="使用する TCP ポート"
          style={{ width: 0 }}
          styles={{ input: { textAlign: 'end', textOverflow: 'clip' } }}
          max={65535}
          min={1}
          // value={String(props.settings.peerCastPort)}
          // onChange={(_ev, newValue) =>
          //   props.onChange({
          //     ...props.settings,
          //     peerCastPort: Number(newValue),
          //   })
          // }
          value={''}
        />
        <TextField
          label="PeerCast から見たこのマシンのアドレス"
          placeholder="localhost"
        />
        <TextField
          label="PeerCast のアドレスと TCP ポート番号"
          placeholder="localhost:7144"
        />
      </div>
      <div
        css={css`
          flex-grow: 1;
          overflow: hidden;
        `}
      >
        <ConnectionsView connections={connections} />
      </div>
    </div>
  );
}
