import type { FC, PropsWithChildren } from 'react';
export type CollapseProps = PropsWithChildren<{
    expanded: boolean;
    id: string;
}>;
export declare const Collapse: FC<CollapseProps>;
