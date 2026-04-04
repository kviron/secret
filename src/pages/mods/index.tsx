import { Component } from 'solid-js';

export const ModsPage: Component = () => {
  return (
    <>
      <header class="top-bar">
        <h1 class="page-title">All Mods</h1>
      </header>
      <div class="empty-state">
        <div class="empty-icon">📦</div>
        <h2>Mod management</h2>
        <p>Select a game from the library to manage its mods.</p>
      </div>
    </>
  );
};