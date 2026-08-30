local function one_cost_cards(ctx, player)
    local result = {}
    local player_class = ctx:player(player).class
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.cost == 1
            and (definition.class == "neutral" or definition.class == player_class) then
            result[#result + 1] = card_id
        end
    end
    return result
end

local card = {
    api_version = 1,
    id = "LOE_023",
    name = "Dark Peddler",
    text = "<b>Battlecry: Discover</b> a\n1-Cost card.",
    set = "LOE",
    type = "minion",
    class = "warlock",
    rarity = "common",
    cost = 2,
    attack = 2,
    health = 3,
    tags = { "undead" },
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local prompt = ctx:localize(
        "Discover a 1-Cost card",
        "发现一张法力值消耗为（1）的牌",
        "發現一張消耗（1）點法力的牌"
    )
    ctx:discover_cards(player, prompt, one_cost_cards(ctx, player), 3, "on_discovered")
end

function card.on_discovered(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

return card
