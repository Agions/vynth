export type Listener<T> = (payload: T) => void;

export class Emitter<M extends Record<string, unknown>> {
  private map = new Map<keyof M, Set<Listener<unknown>>>();

  on<K extends keyof M>(key: K, fn: Listener<M[K]>): () => void {
    let set = this.map.get(key);
    if (!set) {
      set = new Set();
      this.map.set(key, set);
    }
    set.add(fn as Listener<unknown>);
    return () => set?.delete(fn as Listener<unknown>);
  }

  emit<K extends keyof M>(key: K, payload: M[K]): void {
    const set = this.map.get(key);
    if (!set) return;
    for (const fn of set) (fn as Listener<M[K]>)(payload);
  }
}
