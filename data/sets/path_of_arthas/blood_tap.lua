local card = {
    api_version = 1,
    id = "RLK_712",
    name = "Blood Tap",
    text = "Give all minions in your hand +1/+1.\nSpend 2 <b>Corpses</b> to give them +1/+1 more.",
    set = "PATH_OF_ARTHAS",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    spell_school = "shadow",
    cost = 2,
    rune_cost = { blood = 1 },
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local extra = ctx:spend_corpses(player, 2) and 1 or 0
    for _, entity in ipairs(ctx:hand(player)) do
        if ctx:entity(entity).type == "minion" then ctx:buff(entity, 1 + extra, 1 + extra) end
    end
end

return card
