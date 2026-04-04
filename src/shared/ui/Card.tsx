import { Component, JSX, splitProps } from 'solid-js';

interface CardProps {
  children: JSX.Element;
  class?: string;
  onClick?: () => void;
  hoverable?: boolean;
}

export const Card: Component<CardProps> = (props) => {
  const [local, others] = splitProps(props, ['class', 'children', 'hoverable']);
  return (
    <div 
      class={`card ${local.hoverable !== false ? 'card-hoverable' : ''} ${local.class ?? ''}`} 
      onClick={props.onClick} 
      {...others}
    >
      {local.children}
    </div>
  );
};
