local card = {
    api_version = 1, id = "LOE_003", name = "Ethereal Conjurer",
    text = "<b>Battlecry: Discover</b> a spell.",
    set = "LOE", type = "minion", class = "mage", rarity = "common",
    cost = 5, attack = 6, health = 4, keywords = { "battlecry", "discover" },
}

function card.on_battlecry(ctx, self)
    local player = ctx:controller(self)
    local class = ctx:player(player).class
    local pool = {}
    for _, card_id in ipairs(ctx:collectible_cards()) do
        local definition = ctx:card_definition(card_id)
        if definition.type == "spell"
            and (definition.class == "neutral" or definition.class == class) then
            pool[#pool + 1] = card_id
        end
    end
    ctx:discover_cards(player, "Discover a spell", pool, 3, "receive_spell")
end

function card.receive_spell(ctx, self, card_id) ctx:give_card(ctx:controller(self), card_id) end
return card
