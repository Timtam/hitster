import { useCallback, useEffect } from "react"
import { useNavigate } from "react-router-dom"

export const useRevalidate = () => {
    let navigate = useNavigate()
    return useCallback(
        function revalidate() {
            navigate(".", { replace: true })
        },
        [navigate],
    )
}

interface IntervalOptions {
    enabled?: boolean
    interval?: number
}

export const useRevalidateOnInterval = ({
    enabled = false,
    interval = 1000,
}: IntervalOptions) => {
    let revalidate = useRevalidate()
    useEffect(
        function revalidateOnInterval() {
            if (!enabled) return
            let intervalId = setInterval(revalidate, interval)
            return () => clearInterval(intervalId)
        },
        [revalidate],
    )
}
