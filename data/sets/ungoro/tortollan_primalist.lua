local card = {
    api_version = 1, id = "UNG_088", name = "Tortollan Primalist",
    text = "<b>Battlecry:</b> <b>Discover</b> a spell and cast it with random targets.",
    set = "UNGORO", type = "minion", rarity = "epic", cost = 8, attack = 5, health = 5,
    keywords = { "battlecry" },
}
function card.on_battlecry(ctx, self)
    local player, pool = ctx:controller(self), {}
    local own_class = ctx:player(player).class
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        local eligible = definition.class == "neutral" or definition.class == own_class
        if definition.classes and #definition.classes > 0 then
            eligible = false
            for _, class in ipairs(definition.classes) do if class == own_class then eligible = true end end
        end
        if definition.type == "spell" and eligible then pool[#pool + 1] = id end
    end
    if #pool > 0 then ctx:discover_cards(player, "Choose a spell to cast", pool, 3, "cast_primalist_spell") end
end
function card.cast_primalist_spell(ctx, self, id)
    ctx:cast_spell_random_target(ctx:controller(self), id)
end
return card
