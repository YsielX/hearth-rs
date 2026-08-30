local card = {
    api_version = 1,
    id = "RLK_015",
    name = "Howling Blast",
    text = "[x]Deal $3 damage to an\nenemy and <b>Freeze</b> it.\nDeal $1 damage to all\nother enemies.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "common",
    spell_school = "frost",
    cost = 3,
    rune_cost = { frost = 1 },
    target_mode = "required",
    targets = function(ctx, self) return ctx:enemy_characters(self) end,
}

function card.on_play(ctx, self, target)
    local hits = {}
    for _, enemy in ipairs(ctx:enemy_characters(self)) do
        hits[#hits + 1] = { enemy, enemy == target and 3 or 1 }
    end
    cardlib.effects.damage_batch(ctx, hits)
    ctx:freeze(target)
end

return card
