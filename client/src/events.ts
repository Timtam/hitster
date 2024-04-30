export enum Sfx {
    noInterception,
    payToken,
    playHit,
    stopHit,
    youFail,
    youLose,
    youScore,
    youWin,
}

export interface PlaySfxData {
    sfx: Sfx
}

export interface SfxEndedData {
    sfx: Sfx
}

export enum Events {
    playSfx = "Play sfx",
    sfxEnded = "Sfx ended",
}
