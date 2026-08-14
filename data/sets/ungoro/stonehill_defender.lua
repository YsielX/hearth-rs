local function eligible(ctx, player, definition)
    local own = ctx:player(player).class
    if definition.class == "neutral" or definition.class == own then return true end
    for _, class in ipairs(definition.classes or {}) do if class == own then return true end end
    return false
end

local function has_taunt(definition)
    for _, keyword in ipairs(definition.keywords or {}) do if keyword == "taunt" then return true end end
    return false
end

local card = {
    api_version = 1, id = "UNG_072", name = "Stonehill Defender",
    text = "<b>Taunt</b>\n<b>Battlecry:</b> <b>Discover</b> a <b>Taunt</b> minion.",
    set = "UNGORO", type = "minion", rarity = "rare", cost = 3, attack = 1, health = 5,
    keywords = { "taunt", "battlecry", "discover" },
}
function card.on_battlecry(ctx, self)
    local player, pool = ctx:controller(self), {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion" and has_taunt(definition) and eligible(ctx, player, definition) then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then ctx:discover_cards(player, "Discover a Taunt minion", pool, 3, "receive_taunt") end
end
function card.receive_taunt(ctx, self, card_id) ctx:give_card(ctx:controller(self), card_id) end
return card
