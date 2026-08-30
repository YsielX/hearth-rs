local card = {
    api_version = 1, id = "CFM_324", name = "White Eyes",
    text = "<b>Taunt</b>\n<b>Deathrattle:</b> Shuffle\n'The Storm Guardian' into your deck.",
    set = "GANGS", type = "minion", class = "shaman", rarity = "legendary",
    cost = 5, attack = 5, health = 5, keywords = { "taunt", "deathrattle" },
}
function card.on_deathrattle(ctx, self) cardlib.effects.shuffle_card_into_deck(ctx, ctx:controller(self), "CFM_324t") end
card.tokens = {{
    id = "CFM_324t", name = "The Storm Guardian", text = "<b>Taunt</b>", set = "GANGS",
    type = "minion", class = "shaman", cost = 5, attack = 10, health = 10, keywords = { "taunt" },
}}
return card
