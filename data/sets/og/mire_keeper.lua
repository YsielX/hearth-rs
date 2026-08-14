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
        { label = "Summon a 2/2 Slime", value = 1 },
        { label = "Gain an empty Mana Crystal", value = 2 },
    }, "chosen")
end
function card.chosen(ctx, self, choice)
    if choice == 1 then slime(ctx, self) else mana(ctx, self) end
end
function card.on_choose_multiple(ctx, self) slime(ctx, self); mana(ctx, self) end
card.tokens = {{ id = "OG_202c", name = "Slime", text = "", set = "OG",
    type = "minion", class = "druid", cost = 2, attack = 2, health = 2 }}
return card
