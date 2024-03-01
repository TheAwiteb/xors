import { configureStore } from "@reduxjs/toolkit";
import oneVsOne from "./slices/oneVsone";
import ai from "./slices/ai";

export const store = configureStore({
  reducer: {
    oneVsOne,
    ai,
  },
});
