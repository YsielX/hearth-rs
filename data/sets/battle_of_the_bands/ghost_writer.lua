local card = {
    api_version = 1,
    id = "ETC_088",
    name = "Ghost Writer",
    text = "<b>Battlecry:</b> <b>Discover</b> a spell. <b>Finale:</b> <b>Discover</b> another.",
    set = "BATTLE_OF_THE_BANDS",
    type = "minion",
    cost = 5,
    attack = 4,
    health = 4,
    tags = { "undead" },
    keywords = { "battlecry", "finale" },
}

local function spells(ctx, player)
    local result = {}
    local player_class = ctx:player(player).class
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "spell"
            and (definition.class == "neutral" or definition.class == player_class) then
            result[#result + 1] = card_id
        end
    end
    return result
end

card.on_battlecry = function(ctx, self)
    local player = ctx:controller(self)
    local prompt = ctx:localize(
        "Discover a spell",
        "发现一张法术牌",
        "發現一張法術牌"
    )
    ctx:discover_cards(player, prompt, spells(ctx, player), 3, "on_discovered")
end

card.on_finale = function(ctx, self)
    local player = ctx:controller(self)
    local prompt = ctx:localize(
        "Finale: Discover another spell",
        "压轴：再发现一张法术牌",
        "壓軸：再發現一張法術牌"
    )
    ctx:discover_cards(player, prompt, spells(ctx, player), 3, "on_discovered")
end

card.on_discovered = function(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
