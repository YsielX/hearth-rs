local card = {
    api_version = 1,
    id = "CFM_308",
    name = "Kun the Forgotten King",
    text = "<b>Choose One -</b> Gain 10 Armor; or Refresh your Mana Crystals.",
    set = "GANGS",
    type = "minion",
    class = "druid",
    rarity = "legendary",
    cost = 10,
    attack = 7,
    health = 7,
    tags = { "undead" },
    keywords = { "choose_one" },
}

local function armor(ctx, self) ctx:gain_armor(ctx:controller(self), 10) end
local function refresh(ctx, self) ctx:refresh_mana_crystals(ctx:controller(self)) end

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { card_id = "CFM_308a", label = "Gain 10 Armor" },
        { card_id = "CFM_308b", label = "Refresh your Mana Crystals" },
    }, "chosen")
end

function card.chosen(ctx, self, choice)
    if choice == "CFM_308a" then armor(ctx, self) else refresh(ctx, self) end
end

function card.on_choose_multiple(ctx, self)
    armor(ctx, self)
    refresh(ctx, self)
end

card.tokens = {
    { id = "CFM_308a", name = "Forgotten Armor", text = "Gain 10 Armor.", set = "GANGS", type = "spell", class = "druid", collectible = false, cost = 10 },
    { id = "CFM_308b", name = "Forgotten Mana", text = "Refresh your Mana Crystals.", set = "GANGS", type = "spell", class = "druid", collectible = false, cost = 10 },
}

return card
