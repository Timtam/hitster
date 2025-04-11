import { useCallback, useEffect, useState } from "react"
import { useNavigate } from "react-router"

export const useRevalidate = () => {
    const navigate = useNavigate()
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
    const revalidate = useRevalidate()
    useEffect(
        function revalidateOnInterval() {
            if (!enabled) return
            const intervalId = setInterval(revalidate, interval)
            return () => clearInterval(intervalId)
        },
        [enabled, interval, revalidate],
    )
}

export const useModalShown = (): boolean => {
    const [shown, setShown] = useState(false)

    useEffect(() => {
        const id = setInterval(() => {
            setShown(document.querySelector(".modal") !== null)
        }, 50)

        return () => {
            clearInterval(id)
        }
    }, [])

    return shown
}
