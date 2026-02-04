const React = window.React;

if (!React) {
  throw new Error("React not found on window. Ensure react UMD is loaded.");
}

export const Fragment = React.Fragment;

export const jsx = (type, props, key) => {
  const nextProps = props ? { ...props, key } : { key };
  return React.createElement(type, nextProps);
};

export const jsxs = jsx;
export const jsxDEV = jsx;
