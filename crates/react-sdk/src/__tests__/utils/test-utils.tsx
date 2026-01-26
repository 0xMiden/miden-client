import React, { type ReactNode } from "react";
import { render, type RenderOptions, renderHook } from "@testing-library/react";
import { useMidenStore } from "../../store/MidenStore";
import type { MidenConfig } from "../../types";
import {
  createMockWebClient,
  type MockWebClientType,
} from "../mocks/miden-sdk";

// Reset store between tests
export const resetStore = () => {
  useMidenStore.getState().reset();
};

// Provider wrapper with mock client already set
interface WrapperProps {
  children: ReactNode;
}

interface TestProviderOptions {
  config?: MidenConfig;
  mockClient?: Partial<MockWebClientType>;
  initialReady?: boolean;
}

// Create a test provider that sets the client directly (bypassing async init)
export const createTestProvider = (options: TestProviderOptions = {}) => {
  const mockClient = createMockWebClient(options.mockClient);

  const TestProvider = ({ children }: WrapperProps) => {
    // Set up the store directly with our mock client
    React.useEffect(() => {
      if (options.initialReady !== false) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        useMidenStore.getState().setClient(mockClient as any);
      }
    }, []);

    return <>{children}</>;
  };

  return { TestProvider, mockClient };
};

// Render hook with test provider
export const renderHookWithProvider = <TResult, TProps>(
  hook: (props: TProps) => TResult,
  options: TestProviderOptions & { hookProps?: TProps } = {}
) => {
  const { TestProvider, mockClient } = createTestProvider(options);

  const result = renderHook(hook, {
    wrapper: TestProvider,
    initialProps: options.hookProps as TProps,
  });

  return { ...result, mockClient };
};

// Render component with provider
export const renderWithProvider = (
  ui: React.ReactElement,
  options: TestProviderOptions & Omit<RenderOptions, "wrapper"> = {}
): ReturnType<typeof render> & { mockClient: MockWebClientType } => {
  const { TestProvider, mockClient } = createTestProvider(options);

  const result = render(ui, {
    wrapper: TestProvider,
    ...options,
  });

  return { ...result, mockClient };
};

// Wait for async state updates
export const waitForStateUpdate = () =>
  new Promise((resolve) => setTimeout(resolve, 0));

// Helper to wait for loading to complete
export const waitForLoading = async (
  getLoadingState: () => boolean,
  timeout: number = 5000
): Promise<void> => {
  const start = Date.now();
  while (getLoadingState() && Date.now() - start < timeout) {
    await waitForStateUpdate();
  }
};

// Helper to set up store with mock data
export const setupStoreWithData = (options: {
  client?: MockWebClientType;
  accounts?: ReturnType<
    typeof import("../mocks/miden-sdk").createMockAccountHeader
  >[];
  notes?: ReturnType<
    typeof import("../mocks/miden-sdk").createMockInputNoteRecord
  >[];
  syncHeight?: number;
  isReady?: boolean;
}) => {
  const store = useMidenStore.getState();

  if (options.client) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    store.setClient(options.client as any);
  }

  if (options.accounts) {
    store.setAccounts(options.accounts as unknown as typeof store.accounts);
  }

  if (options.notes) {
    store.setNotes(options.notes as unknown as typeof store.notes);
  }

  if (options.syncHeight !== undefined) {
    store.setSyncState({ syncHeight: options.syncHeight });
  }
};
