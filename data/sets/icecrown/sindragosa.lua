local function legendary_minions(ctx)
    local result = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and definition.rarity == "legendary" then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1, id = "ICC_838", name = "Sindragosa",
    text = "<b>Battlecry:</b> Summon two 0/1 Frozen Champions.",
    set = "ICECROWN", type = "minion", class = "mage", rarity = "legendary",
    cost = 8, attack = 8, health = 8, tags = { "undead", "dragon" }, keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    ctx:summon(player, "ICC_838t")
    ctx:summon(player, "ICC_838t")
end

card.tokens = {{
    id = "ICC_838t", name = "Frozen Champion",
    text = "[x]<b>Deathrattle:</b> Add a\nrandom <b>Legendary</b> minion\nto your hand.",
    set = "ICECROWN", type = "minion", class = "mage", collectible = false,
    cost = 1, attack = 0, health = 1, keywords = { "deathrattle" },
    on_deathrattle = function(ctx, self)
        local pool = legendary_minions(ctx)
        if #pool > 0 then ctx:random_value(pool, "frozen_champion_chosen") end
    end,
    frozen_champion_chosen = function(ctx, self, card_id)
        ctx:give_card(ctx:controller(self), card_id)
    end,
}}

return card
