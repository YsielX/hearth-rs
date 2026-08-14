local card = {
    api_version = 1, id = "UNG_854", name = "Free From Amber",
    text = "<b>Discover</b> a minion that costs (8) or more. Summon it.",
    set = "UNGORO", type = "spell", class = "priest", rarity = "rare", cost = 7,
}
function card.on_play(ctx, self)
    local player, own_class, pool = ctx:controller(self), ctx:player(ctx:controller(self)).class, {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        local eligible = definition.class == "neutral" or definition.class == own_class
        if definition.classes and #definition.classes > 0 then
            eligible = false
            for _, class in ipairs(definition.classes) do if class == own_class then eligible = true end end
        end
        if definition.type == "minion" and definition.cost >= 8 and eligible then pool[#pool + 1] = id end
    end
    if #pool > 0 then ctx:discover_cards(player, "Choose a minion to summon", pool, 3, "summon_amber_minion") end
end
function card.summon_amber_minion(ctx, self, id) ctx:summon(ctx:controller(self), id) end
return card
