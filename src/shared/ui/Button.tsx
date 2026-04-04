import { Component, JSX, splitProps } from 'solid-js';

interface ButtonProps {
  onClick?: () => void;
  disabled?: boolean;
  isLoading?: boolean;
  variant?: 'primary' | 'secondary' | 'danger' | 'ghost';
  size?: 'sm' | 'md' | 'lg';
  children: JSX.Element;
  class?: string;
}

export const Button: Component<ButtonProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'variant', 'size', 'isLoading', 'children']);
  const variant = local.variant ?? 'primary';
  const size = local.size ?? 'md';
  
  return (
    <button
      class={`btn btn-${variant} btn-${size} ${local.isLoading ? 'btn-loading' : ''} ${local.class ?? ''}`}
      disabled={local.isLoading || props.disabled}
      {...others}
    >
      {local.isLoading ? (
        <span class="btn-spinner" />
      ) : (
        local.children
      )}
    </button>
  );
};
