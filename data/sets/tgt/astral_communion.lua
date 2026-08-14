local card = {
    api_version = 1, id = "AT_043", name = "Astral Communion",
    text = "Gain 10 Mana Crystals. Discard your hand.",
    set = "TGT", type = "spell", class = "druid", rarity = "epic",
    spell_school = "arcane", cost = 5,
}

function card.on_play(ctx, self)
    local player = ctx:controller(self)
    local at_maximum = ctx:player(player).max_mana >= 10
    ctx:fill_mana_crystals(player, 10)
    if at_maximum then ctx:give_card(player, "CS2_013t") end
    for _, entity in ipairs(ctx:hand(player)) do ctx:discard(player, entity) end
end

return card
