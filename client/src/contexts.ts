import { useOutletContext } from "react-router-dom"
import type { User } from "./entities"

export type UserContext = { user: User | null }

export function useUser() {
    return useOutletContext<UserContext>()
}
