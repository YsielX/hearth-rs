local card = {
    api_version = 1, id = "UNG_963", name = "Lyra the Sunshard",
    text = "Whenever you cast a spell, add a random Priest spell to your hand.",
    set = "UNGORO", type = "minion", class = "priest", rarity = "legendary",
    cost = 4, attack = 3, health = 5, tags = { "elemental" },
}
local function priest_spells(ctx)
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        local eligible = definition.class == "priest"
        for _, class in ipairs(definition.classes or {}) do if class == "priest" then eligible = true end end
        if definition.type == "spell" and eligible then pool[#pool + 1] = id end
    end
    return pool
end
card.triggers = {{
    event = "spell_cast", timing = "after", active_zones = { "board" },
    condition = function(ctx, self, event)
        return event.player == ctx:controller(self) and event.player_cast
    end,
    effect = function(ctx, self)
        local pool = priest_spells(ctx)
        if #pool > 0 then ctx:random_value(pool, "add_lyra_spell") end
    end,
}}
function card.add_lyra_spell(ctx, self, id) ctx:give_card(ctx:controller(self), id) end
return card
