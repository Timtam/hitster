import type { Hit, Slot } from "../../entities"

export const isSlotCorrect = (hit: Hit | null, slot: Slot | null): boolean => {
    if (hit === null || slot === null) return false
    return (
        (slot.from_year === 0 && hit.year <= slot.to_year) ||
        (slot.to_year === 0 && hit.year >= slot.from_year) ||
        (slot.from_year <= hit.year && hit.year <= slot.to_year)
    )
}
