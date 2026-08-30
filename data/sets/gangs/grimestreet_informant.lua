local wanted = { hunter = true, paladin = true, warrior = true }

local function belongs_to_goons(definition)
    if wanted[definition.class] then return true end
    for _, class in ipairs(definition.classes or {}) do
        if wanted[class] then return true end
    end
    return false
end

local card = {
    api_version = 1,
    id = "CFM_321",
    name = "Grimestreet Informant",
    text = "[x]<b>Battlecry:</b> <b>Discover</b>\na Hunter, Paladin, or\nWarrior card.",
    set = "GANGS",
    type = "minion",
    classes = { "hunter", "paladin", "warrior" },
    rarity = "rare",
    cost = 2,
    attack = 2,
    health = 2,
    keywords = { "battlecry", "discover" },
}

function card.on_battlecry(ctx, self)
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        if belongs_to_goons(ctx:card_definition(card_id)) then pool[#pool + 1] = card_id end
    end
    if #pool > 0 then
        ctx:discover_cards(ctx:controller(self), "Discover a Grimy Goons card", pool, 3, "receive_card")
    end
end

function card.receive_card(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

return card
