local function elemental(definition)
    for _, tag in ipairs(definition.tags or {}) do if tag == "elemental" or tag == "all" then return true end end
    return false
end
local function eligible(ctx, player, definition)
    if definition.class == "neutral" or definition.class == ctx:player(player).class then return true end
    for _, class in ipairs(definition.classes or {}) do if class == ctx:player(player).class then return true end end
    return false
end
local card = {
    api_version = 1, id = "UNG_816", name = "Servant of Kalimos",
    text = "[x]<b>Battlecry:</b> If you played\nan Elemental last turn,\n <b>Discover</b> an Elemental.",
    set = "UNGORO", type = "minion", rarity = "rare", cost = 5, attack = 5, health = 5,
    tags = { "elemental" }, keywords = { "battlecry", "discover" },
}
function card.on_battlecry(ctx, self)
    local player, played = ctx:controller(self), false
    for _, id in ipairs(ctx:cards_played_last_turn(player)) do if elemental(ctx:card_definition(id)) then played = true break end end
    if not played then return end
    local pool = {}
    for _, id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(id)
        if definition.type == "minion" and elemental(definition) and eligible(ctx, player, definition) then pool[#pool + 1] = id end
    end
    if #pool > 0 then ctx:discover_cards(player, "Discover an Elemental", pool, 3, "receive_elemental") end
end
function card.receive_elemental(ctx, self, id) cardlib.effects.give_card(ctx, ctx:controller(self), id) end
return card
