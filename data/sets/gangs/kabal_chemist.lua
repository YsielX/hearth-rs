local potions = {
    "CFM_021", "CFM_603", "CFM_661", "CFM_611", "CFM_620",
    "CFM_065", "CFM_608", "CFM_604", "CFM_662", "CFM_094",
}

local card = {
    api_version = 1,
    id = "CFM_619",
    name = "Kabal Chemist",
    text = "<b>Battlecry:</b> Add a random Potion to your hand.",
    set = "GANGS",
    type = "minion",
    classes = { "mage", "priest", "warlock" },
    rarity = "common",
    cost = 4,
    attack = 3,
    health = 3,
    keywords = { "battlecry" },
}

function card.on_battlecry(ctx)
    ctx:random_value(potions, "receive_potion")
end

function card.receive_potion(ctx, self, card_id)
    cardlib.effects.give_card(ctx, ctx:controller(self), card_id)
end

return card
