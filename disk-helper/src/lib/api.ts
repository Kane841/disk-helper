import { mockApi } from "@/mocks/mock-api";

/** M1 prototype: always use mock API. M2+ will switch on VITE_USE_MOCK. */
export const api = mockApi;
