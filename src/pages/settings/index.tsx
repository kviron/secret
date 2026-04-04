import { Component } from 'solid-js';

export const SettingsPage: Component = () => {
  return (
    <>
      <header class="top-bar">
        <h1 class="page-title">Settings</h1>
      </header>
      <div class="empty-state">
        <div class="empty-icon">⚙️</div>
        <h2>Settings</h2>
        <p>Configure Pantheon to suit your needs.</p>
      </div>
    </>
  );
};