import { useOutletContext } from "react-router"
import type { User } from "./entities"

export type Context = {
    user: User | null
    showError: (error: string) => void
}

export function useContext() {
    return useOutletContext<Context>()
}
