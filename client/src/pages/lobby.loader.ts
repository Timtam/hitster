import { Game } from "../entities"
import GameService from "../services/games.service"

const loader = async (): Promise<Game[]> => {
    const gs = new GameService()
    return await gs.getAll()
}

export default loader
