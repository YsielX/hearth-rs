local function elemental(definition)
    for _, tag in ipairs(definition.tags or {}) do if tag == "elemental" or tag == "all" then return true end end
    return false
end
return { api_version = 1, id = "UNG_021", name = "Steam Surger",
    text = "[x]<b>Battlecry:</b> If you played\nan Elemental last turn,\nadd a 'Flame Geyser'\nto your hand.",
    set = "UNGORO", type = "minion", class = "mage", rarity = "rare", cost = 4,
    attack = 5, health = 4, tags = { "elemental" }, keywords = { "battlecry" },
    on_battlecry = function(ctx, self)
        for _, card_id in ipairs(ctx:cards_played_last_turn(ctx:controller(self))) do
            if elemental(ctx:card_definition(card_id)) then ctx:give_card(ctx:controller(self), "UNG_018"); return end
        end
    end }
