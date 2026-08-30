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
    if #minions == 0 then return end
    ctx:spend_resource_and_continue(ctx:controller(self), "corpses", 1, 1, "detonate_paid_corpse")
end

function card.detonate_paid_corpse(ctx, self, spent)
    if spent == 0 then return end
    cardlib.effects.damage_all(ctx, ctx:minions(), 1)
    ctx:continue_with("detonate_next_corpse")
end

return card
