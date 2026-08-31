local card = {
    api_version = 1, id = "OG_047", name = "Feral Rage",
    text = "<b>Choose One -</b> Give your hero +4 Attack this turn; or Gain 8 Armor.",
    set = "OG", type = "spell", class = "druid", rarity = "common", cost = 3,
    keywords = { "choose_one" },
}

local function attack(ctx, self)
    cardlib.effects.buff_until_end_of_turn(ctx, ctx:player(ctx:controller(self)).hero, 4, 0)
end
local function armor(ctx, self) ctx:gain_armor(ctx:controller(self), 8) end

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { card_id = "OG_047a", label = "Give your hero +4 Attack this turn" },
        { card_id = "OG_047b", label = "Gain 8 Armor" },
    }, "chosen")
end
function card.chosen(ctx, self, choice)
    if choice == "OG_047a" then attack(ctx, self) else armor(ctx, self) end
end
function card.on_choose_multiple(ctx, self) attack(ctx, self); armor(ctx, self) end
card.tokens = {
    { id = "OG_047a", name = "Evolve Spines", text = "Give your hero +4 Attack this turn.", set = "OG", type = "spell", class = "druid", collectible = false, cost = 3 },
    { id = "OG_047b", name = "Evolve Scales", text = "Gain 8 Armor.", set = "OG", type = "spell", class = "druid", collectible = false, cost = 3 },
}
return card
