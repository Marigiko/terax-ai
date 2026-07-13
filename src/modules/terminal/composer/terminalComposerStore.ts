import { create } from "zustand";

export type ComposerQueuedItem = {
  id: string;
  text: string;
};

type ComposerState = {
  drafts: Record<number, string>;
  queues: Record<number, ComposerQueuedItem[]>;
  setDraft: (leafId: number, text: string) => void;
  draftFor: (leafId: number) => string;
  consumeDraft: (leafId: number) => string | null;
  enqueueDraft: (leafId: number) => string | null;
  queuedFor: (leafId: number) => ComposerQueuedItem[];
  dequeueById: (leafId: number, id: string) => void;
};

function makeId(): string {
  if (typeof crypto !== "undefined" && "randomUUID" in crypto) {
    return crypto.randomUUID();
  }
  return `q-${Date.now()}-${Math.random().toString(36).slice(2, 10)}`;
}

export const useTerminalComposerStore = create<ComposerState>((set, get) => ({
  drafts: {},
  queues: {},

  setDraft(leafId, text) {
    set((state) => ({
      drafts: { ...state.drafts, [leafId]: text },
    }));
  },

  draftFor(leafId) {
    return get().drafts[leafId] ?? "";
  },

  consumeDraft(leafId) {
    const text = get().drafts[leafId];
    if (text == null || text.trim().length === 0) return null;
    set((state) => {
      const { [leafId]: _, ...rest } = state.drafts;
      const q = state.queues[leafId] ?? [];
      return {
        drafts: rest,
        queues: {
          ...state.queues,
          [leafId]: [...q, { id: makeId(), text }],
        },
      };
    });
    return text;
  },

  enqueueDraft(leafId) {
    const text = get().drafts[leafId];
    if (text == null || text.trim().length === 0) return null;
    set((state) => {
      const q = state.queues[leafId] ?? [];
      return {
        queues: {
          ...state.queues,
          [leafId]: [...q, { id: makeId(), text }],
        },
      };
    });
    return text;
  },

  queuedFor(leafId) {
    return get().queues[leafId] ?? [];
  },

  dequeueById(leafId, id) {
    set((state) => {
      const q = state.queues[leafId] ?? [];
      return {
        queues: {
          ...state.queues,
          [leafId]: q.filter((item) => item.id !== id),
        },
      };
    });
  },
}));
