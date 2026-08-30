local function is_mech(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags) do
        if tag == "mech" then return true end
    end
    return false
end

local card = {
    api_version = 1, id = "LOE_039", name = "Gorillabot A-3",
    text = "<b>Battlecry:</b> If you control another Mech, <b>Discover</b> a Mech.",
    set = "LOE", type = "minion", rarity = "common", cost = 3, attack = 3, health = 4,
    tags = { "mech", "beast" }, keywords = { "battlecry", "discover" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local found = false
    for _, minion in ipairs(ctx:friendly_minions(self)) do
        if minion ~= self and is_mech(ctx, minion) then found = true break end
    end
    if not found then return end

    local class = ctx:player(player).class
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if card_id ~= "LOE_039" and definition.type == "minion"
            and (definition.class == "neutral" or definition.class == class) then
            for _, tag in ipairs(definition.tags) do
                if tag == "mech" then pool[#pool + 1] = card_id break end
            end
        end
    end
    if #pool > 0 then ctx:discover_cards(player, "Discover a Mech", pool, 3, "receive_mech") end
end

function card.receive_mech(ctx, self, card_id) cardlib.effects.give_card(ctx, ctx:controller(self), card_id) end
return card
