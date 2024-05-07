import { JsonPayload } from '../App';

const dummyData: JsonPayload[] = [
  {
    type: 'atom',
    clientHost: 'dummy1',
    serverHost: 'dummy2',
    direction: 'upload',
    payload: { identifier: '#RAW', payload: 'a' },
  },
  {
    type: 'atom',
    clientHost: 'dummy1',
    serverHost: 'dummy2',
    direction: 'download',
    payload: {
      identifier: 'helo',
      children: [
        {
          identifier: 'ida',
          children: [
            {
              identifier: 'idb',
              children: [{ identifier: 'idb', payload: '2' }],
            },
          ],
        },
        { identifier: 'id\0\0', payload: 'https://\nGET\nPROPS' },
      ],
    },
  },
  {
    type: 'atom',
    clientHost: 'dummy1',
    serverHost: 'dummy2',
    direction: 'download',
    payload: {
      identifier: 'helo',
      children: [
        { identifier: 'id\0\0', payload: '124' },
        { identifier: 'id\0\0', payload: '23' },
      ],
    },
  },
  {
    type: 'atom',
    clientHost: 'dummy2',
    serverHost: 'dummy1',
    direction: 'download',

    payload: {
      identifier: 'helo',
      children: [
        { identifier: 'id\0\0', payload: '1234' },
        { identifier: 'id\0\0', payload: '2' },
      ],
    },
  },
  {
    type: 'atom',
    clientHost: 'dummy2',
    serverHost: 'dummy1',
    direction: 'download',
    payload: {
      identifier: 'helo',
      children: [
        { identifier: 'id\0\0', payload: '124' },
        { identifier: 'id\0\0', payload: '23' },
      ],
    },
  },
];

export default dummyData;
