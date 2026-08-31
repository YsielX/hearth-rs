local card = {
    api_version = 1, id = "OG_202", name = "Mire Keeper",
    text = "[x]<b>Choose One -</b> Summon a\n2/2 Slime; or Gain an\nempty Mana Crystal.",
    set = "OG", type = "minion", class = "druid", rarity = "rare",
    cost = 4, attack = 3, health = 4, keywords = { "choose_one" },
}
local function slime(ctx, self) ctx:summon(ctx:controller(self), "OG_202c") end
local function mana(ctx, self) ctx:gain_mana_crystals(ctx:controller(self), 1, false) end
function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { card_id = "OG_202a", label = "Summon a 2/2 Slime" },
        { card_id = "OG_202b", label = "Gain an empty Mana Crystal" },
    }, "chosen")
end
function card.chosen(ctx, self, choice)
    if choice == "OG_202a" then slime(ctx, self) else mana(ctx, self) end
end
function card.on_choose_multiple(ctx, self) slime(ctx, self); mana(ctx, self) end
card.tokens = {
    { id = "OG_202a", name = "Y'Shaarj's Strength", text = "Summon a 2/2 Slime.", set = "OG", type = "spell", class = "druid", collectible = false, cost = 4 },
    { id = "OG_202b", name = "Yogg-Saron's Magic", text = "Gain an empty Mana Crystal.", set = "OG", type = "spell", class = "druid", collectible = false, cost = 4 },
    { id = "OG_202c", name = "Slime", text = "", set = "OG", type = "minion", class = "druid", collectible = false, cost = 2, attack = 2, health = 2 },
}
return card
