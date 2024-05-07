import { css } from '@emotion/react';
import React from 'react';
import ReactDOM from 'react-dom';
import App from './App';
import './index.css';
import initFluentUI from './utils/initFluentUI';

initFluentUI();

ReactDOM.render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
  document.getElementById('root')
);
