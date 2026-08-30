local card = {
    api_version = 1,
    id = "RLK_035",
    name = "Corpse Explosion",
    text = "Detonate a <b>Corpse</b> to deal $1 damage to all minions. If any are still alive, repeat this.",
    set = "RETURN_OF_THE_LICH_KING",
    type = "spell",
    class = "death_knight",
    rarity = "rare",
    spell_school = "shadow",
    cost = 5,
    rune_cost = { blood = 2 },
}

function card.on_play(ctx, self)
    ctx:continue_with("detonate_next_corpse")
end

function card.detonate_next_corpse(ctx, self)
    local minions = ctx:minions()
    local player = ctx:controller(self)
    if #minions == 0 or not ctx:spend_corpses(player, 1) then return end
    cardlib.effects.damage_all(ctx, minions, 1)
    ctx:continue_with("detonate_next_corpse")
end

return card
