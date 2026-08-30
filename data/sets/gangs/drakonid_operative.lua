local card = {
    api_version = 1, id = "CFM_605", name = "Drakonid Operative",
    text = "[x]<b>Battlecry:</b> If you're holding\na Dragon, <b>Discover</b> a\ncopy of a card in your\nopponent's deck.",
    set = "GANGS", type = "minion", class = "priest", rarity = "rare",
    cost = 4, attack = 4, health = 5, tags = { "dragon" }, keywords = { "battlecry" },
}
local function dragon(ctx, entity)
    for _, tag in ipairs(ctx:card_definition(ctx:entity(entity).card_id).tags or {}) do
        if tag == "dragon" or tag == "all" then return true end
    end
    return false
end
function card.on_battlecry(ctx, self)
    local holding = false
    for _, entity in ipairs(ctx:hand(ctx:controller(self))) do if dragon(ctx, entity) then holding = true break end end
    if not holding then return end
    local deck = ctx:deck(ctx:opponent(ctx:controller(self)))
    if #deck > 0 then
        ctx:discover_entities(ctx:controller(self), "Choose a card to copy", deck, 3, "copy_drakonid_choice")
    end
end
function card.copy_drakonid_choice(ctx, self, chosen)
    cardlib.effects.give_card(ctx, ctx:controller(self), ctx:entity(chosen).card_id)
end
return card
