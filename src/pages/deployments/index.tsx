import { Component } from 'solid-js';

export const DeploymentsPage: Component = () => {
  return (
    <>
      <header class="top-bar">
        <h1 class="page-title">Deployments</h1>
      </header>
      <div class="empty-state">
        <div class="empty-icon">🚀</div>
        <h2>Deployments</h2>
        <p>Manage mod deployments across your game library.</p>
      </div>
    </>
  );
};