local function has_tag(definition, wanted)
    for _, tag in ipairs(definition.tags or {}) do
        if tag == wanted or tag == "all" then return true end
    end
    return false
end

local function holding_dragon(ctx, self)
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do
        if entity ~= self and has_tag(ctx:card_definition(ctx:entity(entity).card_id), "dragon") then
            return true
        end
    end
    return false
end

local card = {
    api_version = 1,
    id = "KAR_062",
    name = "Netherspite Historian",
    text = "<b>Battlecry:</b> If you're holding a Dragon, <b>Discover</b>\na Dragon.",
    set = "KARA",
    type = "minion",
    rarity = "common",
    cost = 2,
    attack = 2,
    health = 3,
    keywords = { "battlecry", "discover" },
}

function card.on_battlecry(ctx, self)
    if not holding_dragon(ctx, self) then return end
    local player = ctx:controller(self)
    local player_class = ctx:player(player).class
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "minion"
            and has_tag(definition, "dragon")
            and (definition.class == "neutral" or definition.class == player_class) then
            pool[#pool + 1] = card_id
        end
    end
    if #pool > 0 then
        ctx:discover_cards(player, "Discover a Dragon", pool, 3, "receive_dragon")
    end
end

function card.receive_dragon(ctx, self, card_id)
    ctx:give_card(ctx:controller(self), card_id)
end

return card
