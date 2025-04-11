import { createContext, type RefObject } from "react"
import { ToastsHandle } from "./types"

const ToastsContext = createContext<RefObject<ToastsHandle | null> | undefined>(
    undefined,
)
export default ToastsContext
